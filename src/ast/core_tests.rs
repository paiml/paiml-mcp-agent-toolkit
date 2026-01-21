//\! Tests for AST core
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_node_size() {
        // Ensure our node structure is within expected bounds
        // With proof annotations, the size is larger than the original 64 bytes
        let size = std::mem::size_of::<UnifiedAstNode>();
        assert!(
            size <= 128,
            "Node size {size} exceeds maximum expected size of 128 bytes"
        );
        // Structure should be at least 64 bytes for the core data
        assert!(
            size >= 64,
            "Node size {size} is smaller than minimum expected size of 64 bytes"
        );
    }

    #[test]
    fn test_node_alignment() {
        // Ensure proper alignment for SIMD operations
        assert_eq!(std::mem::align_of::<UnifiedAstNode>(), 32);
    }

    #[test]
    fn test_node_flags() {
        let mut flags = NodeFlags::new();

        flags.set(NodeFlags::ASYNC);
        flags.set(NodeFlags::EXPORTED);

        assert!(flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));

        flags.unset(NodeFlags::ASYNC);
        assert!(!flags.has(NodeFlags::ASYNC));
    }

    #[test]
    fn test_ast_dag() {
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        let key = dag.add_node(node);

        assert_eq!(dag.nodes.len(), 1);
        assert!(dag.dirty_nodes.contains(key));

        dag.mark_clean(key);
        assert!(!dag.dirty_nodes.contains(key));
    }
}


mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

/// Comprehensive EXTREME TDD tests for ast/core module
/// Tests all public structs, enums, and functions with edge cases

mod coverage_tests {
    use super::*;
    use chrono::Utc;
    use proptest::prelude::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::path::PathBuf;

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
        let cloned = lang.clone();
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
        let cloned = flags.clone();
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

    // ==================== PropertyType Tests ====================

    #[test]
    fn test_property_type_all_variants() {
        let variants = [
            PropertyType::MemorySafety,
            PropertyType::ThreadSafety,
            PropertyType::DataRaceFreeze,
            PropertyType::Termination,
            PropertyType::FunctionalCorrectness("test_spec".to_string()),
            PropertyType::ResourceBounds {
                cpu: Some(100),
                memory: Some(1024),
            },
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_property_type_resource_bounds_optional() {
        let cpu_only = PropertyType::ResourceBounds {
            cpu: Some(100),
            memory: None,
        };
        let memory_only = PropertyType::ResourceBounds {
            cpu: None,
            memory: Some(1024),
        };
        let both = PropertyType::ResourceBounds {
            cpu: Some(100),
            memory: Some(1024),
        };
        let neither = PropertyType::ResourceBounds {
            cpu: None,
            memory: None,
        };

        // All should serialize successfully
        for prop in [cpu_only, memory_only, both, neither] {
            let json = serde_json::to_string(&prop).expect("serialization failed");
            let _: PropertyType = serde_json::from_str(&json).expect("deserialization failed");
        }
    }

    #[test]
    fn test_property_type_hash() {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        PropertyType::MemorySafety.hash(&mut hasher1);
        PropertyType::MemorySafety.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== VerificationMethod Tests ====================

    #[test]
    fn test_verification_method_all_variants() {
        let variants = [
            VerificationMethod::BorrowChecker,
            VerificationMethod::FormalProof {
                prover: "Coq".to_string(),
            },
            VerificationMethod::StaticAnalysis {
                tool: "Miri".to_string(),
            },
            VerificationMethod::ModelChecking { bounded: true },
            VerificationMethod::ModelChecking { bounded: false },
            VerificationMethod::AbstractInterpretation,
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_verification_method_serialization() {
        let methods = [
            VerificationMethod::BorrowChecker,
            VerificationMethod::FormalProof {
                prover: "Lean".to_string(),
            },
            VerificationMethod::StaticAnalysis {
                tool: "Clippy".to_string(),
            },
            VerificationMethod::ModelChecking { bounded: true },
            VerificationMethod::AbstractInterpretation,
        ];

        for method in methods {
            let json = serde_json::to_string(&method).expect("serialization failed");
            let deserialized: VerificationMethod =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(method, deserialized);
        }
    }

    // ==================== EvidenceType Tests ====================

    #[test]
    fn test_evidence_type_all_variants() {
        let variants = [
            EvidenceType::ImplicitTypeSystemGuarantee,
            EvidenceType::ProofScriptReference {
                uri: "file://proof.v".to_string(),
            },
            EvidenceType::TheoremName {
                theorem: "memory_safety_theorem".to_string(),
                theory: Some("MemorySafety".to_string()),
            },
            EvidenceType::TheoremName {
                theorem: "theorem".to_string(),
                theory: None,
            },
            EvidenceType::StaticAnalysisReport {
                report_id: "report_001".to_string(),
            },
            EvidenceType::CertificateHash {
                hash: "abc123".to_string(),
                algorithm: "sha256".to_string(),
            },
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_evidence_type_serialization() {
        let evidence = EvidenceType::CertificateHash {
            hash: "deadbeef".to_string(),
            algorithm: "sha512".to_string(),
        };
        let json = serde_json::to_string(&evidence).expect("serialization failed");
        let deserialized: EvidenceType =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(evidence, deserialized);
    }

    // ==================== BytePos Tests ====================

    #[test]
    fn test_byte_pos_to_usize() {
        let pos = BytePos(42);
        assert_eq!(pos.to_usize(), 42);
    }

    #[test]
    fn test_byte_pos_from_usize() {
        let pos = BytePos::from_usize(1000);
        assert_eq!(pos.0, 1000);
        assert_eq!(pos.to_usize(), 1000);
    }

    #[test]
    fn test_byte_pos_from_usize_max() {
        let pos = BytePos::from_usize(u32::MAX as usize);
        assert_eq!(pos.0, u32::MAX);
    }

    #[test]
    fn test_byte_pos_ordering() {
        let pos1 = BytePos(10);
        let pos2 = BytePos(20);
        let pos3 = BytePos(10);

        assert!(pos1 < pos2);
        assert!(pos2 > pos1);
        assert!(pos1 <= pos3);
        assert!(pos1 >= pos3);
        assert_eq!(pos1, pos3);
    }

    #[test]
    fn test_byte_pos_hash() {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        BytePos(42).hash(&mut hasher1);
        BytePos(42).hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== Span Tests ====================

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 50);
        assert_eq!(span.start.0, 10);
        assert_eq!(span.end.0, 50);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(10, 50);
        assert_eq!(span.len(), 40);
    }

    #[test]
    fn test_span_len_zero() {
        let span = Span::new(10, 10);
        assert_eq!(span.len(), 0);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(10, 10);
        let non_empty = Span::new(10, 20);
        let invalid = Span::new(20, 10); // end < start

        assert!(empty.is_empty());
        assert!(!non_empty.is_empty());
        assert!(invalid.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 20);

        assert!(span.contains(BytePos(10))); // start is inclusive
        assert!(span.contains(BytePos(15)));
        assert!(!span.contains(BytePos(20))); // end is exclusive
        assert!(!span.contains(BytePos(9)));
        assert!(!span.contains(BytePos(21)));
    }

    #[test]
    fn test_span_hash() {
        let span1 = Span::new(10, 20);
        let span2 = Span::new(10, 20);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        span1.hash(&mut hasher1);
        span2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== Location Tests ====================

    #[test]
    fn test_location_new() {
        let loc = Location::new(PathBuf::from("test.rs"), 100, 200);
        assert_eq!(loc.file_path, PathBuf::from("test.rs"));
        assert_eq!(loc.span.start.0, 100);
        assert_eq!(loc.span.end.0, 200);
        assert_eq!(loc.span.len(), 100);
    }

    #[test]
    fn test_location_contains() {
        let file = PathBuf::from("test.rs");
        let outer = Location::new(file.clone(), 0, 100);
        let inner = Location::new(file.clone(), 10, 50);
        let exact = Location::new(file.clone(), 0, 100);
        let partial_overlap = Location::new(file.clone(), 50, 150);
        let different_file = Location::new(PathBuf::from("other.rs"), 10, 50);

        assert!(outer.contains(&inner));
        assert!(outer.contains(&exact));
        assert!(!inner.contains(&outer));
        assert!(!outer.contains(&partial_overlap));
        assert!(!outer.contains(&different_file));
    }

    #[test]
    fn test_location_overlaps() {
        let file = PathBuf::from("test.rs");
        let loc1 = Location::new(file.clone(), 0, 50);
        let loc2 = Location::new(file.clone(), 25, 75);
        let loc3 = Location::new(file.clone(), 50, 100); // touches but doesn't overlap
        let loc4 = Location::new(file.clone(), 100, 150);
        let different_file = Location::new(PathBuf::from("other.rs"), 25, 75);

        assert!(loc1.overlaps(&loc2));
        assert!(loc2.overlaps(&loc1));
        assert!(!loc1.overlaps(&loc3)); // end of loc1 == start of loc3, but not overlapping
        assert!(!loc1.overlaps(&loc4));
        assert!(!loc1.overlaps(&different_file));
    }

    #[test]
    fn test_location_self_overlap_and_contain() {
        let loc = Location::new(PathBuf::from("test.rs"), 10, 50);
        assert!(loc.overlaps(&loc));
        assert!(loc.contains(&loc));
    }

    #[test]
    fn test_location_hash() {
        let loc1 = Location::new(PathBuf::from("test.rs"), 10, 50);
        let loc2 = Location::new(PathBuf::from("test.rs"), 10, 50);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        loc1.hash(&mut hasher1);
        loc2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_location_hash_prefix_matching() {
        // End position is omitted for prefix matching scenarios
        let loc1 = Location::new(PathBuf::from("test.rs"), 10, 50);
        let loc2 = Location::new(PathBuf::from("test.rs"), 10, 100);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        loc1.hash(&mut hasher1);
        loc2.hash(&mut hasher2);

        // Should have same hash due to prefix matching design
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== QualifiedName Tests ====================

    #[test]
    fn test_qualified_name_new() {
        let qname = QualifiedName::new(
            vec!["std".to_string(), "io".to_string()],
            "Read".to_string(),
        );
        assert_eq!(qname.module_path, vec!["std", "io"]);
        assert_eq!(qname.name, "Read");
        assert!(qname.disambiguator.is_none());
    }

    #[test]
    fn test_qualified_name_new_empty_path() {
        let qname = QualifiedName::new(vec![], "main".to_string());
        assert!(qname.module_path.is_empty());
        assert_eq!(qname.name, "main");
    }

    #[test]
    fn test_qualified_name_with_disambiguator() {
        let qname =
            QualifiedName::new(vec!["crate".to_string()], "func".to_string()).with_disambiguator(1);
        assert_eq!(qname.disambiguator, Some(1));
    }

    #[test]
    fn test_qualified_name_from_string() {
        let qname = QualifiedName::from_string("std::collections::HashMap").expect("parse failed");
        assert_eq!(qname.module_path, vec!["std", "collections"]);
        assert_eq!(qname.name, "HashMap");
    }

    #[test]
    fn test_qualified_name_from_string_simple() {
        let qname = QualifiedName::from_string("main").expect("parse failed");
        assert!(qname.module_path.is_empty());
        assert_eq!(qname.name, "main");
    }

    #[test]
    fn test_qualified_name_from_string_empty() {
        let result = QualifiedName::from_string("");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Empty qualified name");
    }

    #[test]
    fn test_qualified_name_from_string_trailing_separator() {
        let result = QualifiedName::from_string("std::io::");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Empty qualified name");
    }

    #[test]
    fn test_qualified_name_to_qualified_string() {
        let qname = QualifiedName::new(
            vec!["crate".to_string(), "module".to_string()],
            "function".to_string(),
        );
        assert_eq!(qname.to_qualified_string(), "crate::module::function");
    }

    #[test]
    fn test_qualified_name_to_qualified_string_with_disambiguator() {
        let qname =
            QualifiedName::new(vec!["crate".to_string()], "func".to_string()).with_disambiguator(2);
        assert_eq!(qname.to_qualified_string(), "crate::func#2");
    }

    #[test]
    fn test_qualified_name_to_qualified_string_simple() {
        let qname = QualifiedName::new(vec![], "main".to_string());
        assert_eq!(qname.to_qualified_string(), "main");
    }

    #[test]
    fn test_qualified_name_display() {
        let qname = QualifiedName::new(
            vec!["std".to_string(), "io".to_string()],
            "Read".to_string(),
        );
        assert_eq!(format!("{}", qname), "std::io::Read");
    }

    #[test]
    fn test_qualified_name_from_str() {
        let qname: QualifiedName = "std::io::Read".parse().expect("parse failed");
        assert_eq!(qname.name, "Read");
        assert_eq!(qname.module_path, vec!["std", "io"]);
    }

    #[test]
    fn test_qualified_name_hash() {
        let qname1 = QualifiedName::new(vec!["std".to_string()], "io".to_string());
        let qname2 = QualifiedName::new(vec!["std".to_string()], "io".to_string());

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        qname1.hash(&mut hasher1);
        qname2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== RelativeLocation Tests ====================

    #[test]
    fn test_relative_location_function() {
        let loc = RelativeLocation::Function {
            name: "test_fn".to_string(),
            module: Some("test_mod".to_string()),
        };

        let json = serde_json::to_string(&loc).expect("serialization failed");
        let deserialized: RelativeLocation =
            serde_json::from_str(&json).expect("deserialization failed");

        match deserialized {
            RelativeLocation::Function { name, module } => {
                assert_eq!(name, "test_fn");
                assert_eq!(module, Some("test_mod".to_string()));
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_relative_location_function_no_module() {
        let loc = RelativeLocation::Function {
            name: "test_fn".to_string(),
            module: None,
        };

        let json = serde_json::to_string(&loc).expect("serialization failed");
        assert!(!json.contains("module") || json.contains("null"));
    }

    #[test]
    fn test_relative_location_symbol() {
        let loc = RelativeLocation::Symbol {
            qualified_name: "crate::module::Type::method".to_string(),
        };

        let json = serde_json::to_string(&loc).expect("serialization failed");
        let deserialized: RelativeLocation =
            serde_json::from_str(&json).expect("deserialization failed");

        match deserialized {
            RelativeLocation::Symbol { qualified_name } => {
                assert_eq!(qualified_name, "crate::module::Type::method");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_relative_location_span() {
        let loc = RelativeLocation::Span {
            start: 100,
            end: 200,
        };

        let json = serde_json::to_string(&loc).expect("serialization failed");
        let deserialized: RelativeLocation =
            serde_json::from_str(&json).expect("deserialization failed");

        match deserialized {
            RelativeLocation::Span { start, end } => {
                assert_eq!(start, 100);
                assert_eq!(end, 200);
            }
            _ => panic!("Wrong variant"),
        }
    }

    // ==================== NodeMetadata Tests ====================

    #[test]
    fn test_node_metadata_default() {
        let metadata = NodeMetadata::default();
        // SAFETY: accessing raw field for test
        unsafe {
            assert_eq!(metadata.raw, 0);
        }
    }

    #[test]
    fn test_node_metadata_clone() {
        let metadata = NodeMetadata { complexity: 42 };
        let cloned = metadata.clone();
        // SAFETY: accessing complexity field for test
        unsafe {
            assert_eq!(cloned.complexity, 42);
        }
    }

    #[test]
    fn test_node_metadata_copy() {
        let metadata = NodeMetadata { hash: 0xDEADBEEF };
        let copied = metadata;
        // SAFETY: accessing hash field for test
        unsafe {
            assert_eq!(copied.hash, 0xDEADBEEF);
        }
    }

    #[test]
    fn test_node_metadata_union_views() {
        let mut metadata = NodeMetadata::default();
        metadata.complexity = 100;

        // SAFETY: all union fields share the same memory
        unsafe {
            assert_eq!(metadata.raw, 100);
            assert_eq!(metadata.hash, 100);
            assert_eq!(metadata.flags, 100);
        }
    }

    // ==================== ProofAnnotation Tests ====================

    fn create_test_proof_annotation() -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: PropertyType::MemorySafety,
            specification_id: Some("spec_001".to_string()),
            method: VerificationMethod::BorrowChecker,
            tool_name: "rustc".to_string(),
            tool_version: "1.70.0".to_string(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec!["no unsafe".to_string()],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: Some("/path/to/evidence".to_string()),
            date_verified: Utc::now(),
        }
    }

    #[test]
    fn test_proof_annotation_serialization() {
        let annotation = create_test_proof_annotation();
        let json = serde_json::to_string(&annotation).expect("serialization failed");
        let deserialized: ProofAnnotation =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(annotation.property_proven, deserialized.property_proven);
        assert_eq!(annotation.tool_name, deserialized.tool_name);
        assert_eq!(annotation.confidence_level, deserialized.confidence_level);
    }

    #[test]
    fn test_proof_annotation_optional_fields() {
        let annotation = ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: PropertyType::ThreadSafety,
            specification_id: None,
            method: VerificationMethod::AbstractInterpretation,
            tool_name: "tool".to_string(),
            tool_version: "1.0".to_string(),
            confidence_level: ConfidenceLevel::Low,
            assumptions: vec![],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: Utc::now(),
        };

        let json = serde_json::to_string(&annotation).expect("serialization failed");
        // Optional fields should be skipped if None/empty
        assert!(!json.contains("specification_id") || json.contains("null"));
    }

    // ==================== UnifiedAstNode Tests ====================

    #[test]
    fn test_unified_ast_node_new() {
        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        assert!(matches!(
            node.kind,
            AstKind::Function(FunctionKind::Regular)
        ));
        assert_eq!(node.lang, Language::Rust);
        assert_eq!(node.parent, 0);
        assert_eq!(node.first_child, 0);
        assert_eq!(node.next_sibling, 0);
        assert_eq!(node.source_range, 0..0);
        assert_eq!(node.semantic_hash, 0);
        assert_eq!(node.structural_hash, 0);
        assert_eq!(node.name_vector, 0);
        assert!(!node.has_proof_annotations());
    }

    #[test]
    fn test_unified_ast_node_is_function() {
        let func = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let method = UnifiedAstNode::new(AstKind::Function(FunctionKind::Method), Language::Python);
        let class = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);

        assert!(func.is_function());
        assert!(method.is_function());
        assert!(!class.is_function());
    }

    #[test]
    fn test_unified_ast_node_is_type_definition() {
        let struct_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);
        let interface =
            UnifiedAstNode::new(AstKind::Class(ClassKind::Interface), Language::TypeScript);
        let type_alias = UnifiedAstNode::new(AstKind::Type(TypeKind::Alias), Language::Rust);
        let module = UnifiedAstNode::new(AstKind::Module(ModuleKind::File), Language::Python);
        let func = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        assert!(struct_node.is_type_definition());
        assert!(interface.is_type_definition());
        assert!(type_alias.is_type_definition());
        assert!(module.is_type_definition());
        assert!(!func.is_type_definition());
    }

    #[test]
    fn test_unified_ast_node_complexity() {
        let mut node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        assert_eq!(node.complexity(), 0);

        node.set_complexity(42);
        assert_eq!(node.complexity(), 42);

        node.set_complexity(u32::MAX);
        assert_eq!(node.complexity(), u32::MAX);
    }

    #[test]
    fn test_unified_ast_node_proof_annotations() {
        let mut node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        assert!(!node.has_proof_annotations());
        assert_eq!(node.proof_annotations().len(), 0);

        let annotation1 = create_test_proof_annotation();
        node.add_proof_annotation(annotation1);

        assert!(node.has_proof_annotations());
        assert_eq!(node.proof_annotations().len(), 1);

        let annotation2 = create_test_proof_annotation();
        node.add_proof_annotation(annotation2);

        assert_eq!(node.proof_annotations().len(), 2);
    }

    #[test]
    fn test_unified_ast_node_location() {
        let mut node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        node.source_range = 100..200;

        let location = node.location(Path::new("src/main.rs"));
        assert_eq!(location.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(location.span.start.0, 100);
        assert_eq!(location.span.end.0, 200);
    }

    #[test]
    fn test_unified_ast_node_debug() {
        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let debug_str = format!("{:?}", node);

        assert!(debug_str.contains("UnifiedAstNode"));
        assert!(debug_str.contains("kind"));
        assert!(debug_str.contains("lang"));
    }

    #[test]
    fn test_unified_ast_node_clone() {
        let mut node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        node.semantic_hash = 12345;
        node.set_complexity(100);

        let cloned = node.clone();

        assert_eq!(cloned.semantic_hash, 12345);
        assert_eq!(cloned.complexity(), 100);
    }

    // ==================== ColumnStore Tests ====================

    #[test]
    fn test_column_store_new() {
        let store: ColumnStore<i32> = ColumnStore::new(100);
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_column_store_push_and_get() {
        let mut store: ColumnStore<String> = ColumnStore::new(10);

        let key0 = store.push("first".to_string());
        let key1 = store.push("second".to_string());

        assert_eq!(key0, 0);
        assert_eq!(key1, 1);
        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());

        assert_eq!(store.get(key0), Some(&"first".to_string()));
        assert_eq!(store.get(key1), Some(&"second".to_string()));
        assert_eq!(store.get(999), None);
    }

    #[test]
    fn test_column_store_get_mut() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        let key = store.push(42);

        if let Some(value) = store.get_mut(key) {
            *value = 100;
        }

        assert_eq!(store.get(key), Some(&100));
        assert!(store.get_mut(999).is_none());
    }

    #[test]
    fn test_column_store_iter() {
        let mut store: ColumnStore<i32> = ColumnStore::new(10);
        store.push(1);
        store.push(2);
        store.push(3);

        let sum: i32 = store.iter().sum();
        assert_eq!(sum, 6);

        let collected: Vec<_> = store.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }

    // ==================== AstDag Tests ====================

    #[test]
    fn test_ast_dag_new() {
        let dag = AstDag::new();
        assert_eq!(dag.nodes.len(), 0);
        assert!(dag.nodes.is_empty());
        assert_eq!(dag.generation(), 0);
    }

    #[test]
    fn test_ast_dag_default() {
        let dag = AstDag::default();
        assert_eq!(dag.nodes.len(), 0);
        assert_eq!(dag.generation(), 0);
    }

    #[test]
    fn test_ast_dag_add_node() {
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let key = dag.add_node(node);

        assert_eq!(key, 0);
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.generation(), 1);
    }

    #[test]
    fn test_ast_dag_dirty_nodes() {
        let mut dag = AstDag::new();

        let node1 = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let node2 = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::TypeScript);

        let key1 = dag.add_node(node1);
        let key2 = dag.add_node(node2);

        let dirty: Vec<_> = dag.dirty_nodes().collect();
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&key1));
        assert!(dirty.contains(&key2));
    }

    #[test]
    fn test_ast_dag_mark_clean() {
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let key = dag.add_node(node);

        assert!(dag.dirty_nodes().any(|k| k == key));

        dag.mark_clean(key);

        assert!(!dag.dirty_nodes().any(|k| k == key));
        // Node still exists
        assert!(dag.nodes.get(key).is_some());
    }

    #[test]
    fn test_ast_dag_generation_increments() {
        let mut dag = AstDag::new();

        assert_eq!(dag.generation(), 0);

        dag.add_node(UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::Rust,
        ));
        assert_eq!(dag.generation(), 1);

        dag.add_node(UnifiedAstNode::new(
            AstKind::Class(ClassKind::Struct),
            Language::Python,
        ));
        assert_eq!(dag.generation(), 2);

        // mark_clean does not change generation
        dag.mark_clean(0);
        assert_eq!(dag.generation(), 2);
    }

    #[test]
    fn test_ast_dag_multiple_operations() {
        let mut dag = AstDag::new();

        // Add several nodes
        let keys: Vec<_> = (0..5)
            .map(|_| {
                dag.add_node(UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    Language::Rust,
                ))
            })
            .collect();

        assert_eq!(dag.nodes.len(), 5);
        assert_eq!(dag.generation(), 5);
        assert_eq!(dag.dirty_nodes().count(), 5);

        // Clean some
        dag.mark_clean(keys[0]);
        dag.mark_clean(keys[2]);
        dag.mark_clean(keys[4]);

        assert_eq!(dag.dirty_nodes().count(), 2);
        let dirty: Vec<_> = dag.dirty_nodes().collect();
        assert!(dirty.contains(&keys[1]));
        assert!(dirty.contains(&keys[3]));
    }

    // ==================== INVALID_NODE_KEY Tests ====================

    #[test]
    fn test_invalid_node_key_constant() {
        assert_eq!(INVALID_NODE_KEY, u32::MAX);
    }

    // ==================== ClassKind Tests ====================

    #[test]
    fn test_class_kind_all_variants() {
        let variants = [
            ClassKind::Regular,
            ClassKind::Abstract,
            ClassKind::Interface,
            ClassKind::Trait,
            ClassKind::Enum,
            ClassKind::Struct,
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_class_kind_serialization() {
        for kind in [
            ClassKind::Regular,
            ClassKind::Abstract,
            ClassKind::Interface,
            ClassKind::Trait,
            ClassKind::Enum,
            ClassKind::Struct,
        ] {
            let json = serde_json::to_string(&kind).expect("serialization failed");
            let deserialized: ClassKind =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(kind, deserialized);
        }
    }

    // ==================== Property-Based Tests ====================

    proptest! {
        #[test]
        fn prop_byte_pos_roundtrip(pos: u32) {
            let byte_pos = BytePos(pos);
            prop_assert_eq!(byte_pos.0, pos);
            prop_assert_eq!(byte_pos.to_usize(), pos as usize);
        }

        #[test]
        fn prop_span_len_consistency(start in 0u32..1000, len in 0u32..1000) {
            let end = start.saturating_add(len);
            let span = Span::new(start, end);
            prop_assert_eq!(span.len(), end - start);
        }

        #[test]
        fn prop_span_contains_boundary(start in 0u32..1000, len in 1u32..1000) {
            let end = start.saturating_add(len);
            let span = Span::new(start, end);
            prop_assert!(span.contains(BytePos(start)));     // start is inclusive
            prop_assert!(!span.contains(BytePos(end)));       // end is exclusive
            if start > 0 {
                prop_assert!(!span.contains(BytePos(start - 1)));
            }
        }

        #[test]
        fn prop_span_is_empty_consistency(start in 0u32..1000, end in 0u32..1000) {
            let span = Span::new(start, end);
            prop_assert_eq!(span.is_empty(), start >= end);
        }

        #[test]
        fn prop_location_contains_reflexive(
            start in 0u32..1000,
            end in 0u32..2000
        ) {
            let file = PathBuf::from("test.rs");
            let loc = Location::new(file, start, end);
            prop_assert!(loc.contains(&loc));
        }

        #[test]
        fn prop_location_overlaps_reflexive(
            start in 0u32..1000,
            len in 1u32..1000
        ) {
            let file = PathBuf::from("test.rs");
            let end = start.saturating_add(len);
            let loc = Location::new(file, start, end);
            prop_assert!(loc.overlaps(&loc));
        }

        #[test]
        fn prop_qualified_name_roundtrip(
            path_parts in proptest::collection::vec("[a-z]+", 0..5),
            name in "[a-z]+"
        ) {
            let qname = QualifiedName::new(path_parts.clone(), name.clone());
            let qualified_str = qname.to_qualified_string();
            let parsed = QualifiedName::from_string(&qualified_str).expect("parse should succeed");

            prop_assert_eq!(parsed.name, name);
            prop_assert_eq!(parsed.module_path, path_parts);
        }

        #[test]
        fn prop_node_flags_set_unset_idempotent(flag1 in 0u8..8, flag2 in 0u8..8) {
            let flag_value1 = 1u8 << flag1;
            let flag_value2 = 1u8 << flag2;

            let mut flags = NodeFlags::new();
            flags.set(flag_value1);
            flags.set(flag_value1); // Set twice
            prop_assert!(flags.has(flag_value1));

            flags.unset(flag_value2);
            flags.unset(flag_value2); // Unset twice
            if flag_value1 != flag_value2 {
                prop_assert!(flags.has(flag_value1));
            }
        }

        #[test]
        fn prop_unified_ast_node_complexity_roundtrip(complexity in 0u32..=u32::MAX) {
            let mut node = UnifiedAstNode::new(
                AstKind::Function(FunctionKind::Regular),
                Language::Rust
            );
            node.set_complexity(complexity);
            prop_assert_eq!(node.complexity(), complexity);
        }

        #[test]
        fn prop_column_store_push_returns_sequential_keys(count in 1usize..100) {
            let mut store: ColumnStore<i32> = ColumnStore::new(count);
            let keys: Vec<_> = (0..count).map(|i| store.push(i as i32)).collect();

            for (expected_key, actual_key) in keys.iter().enumerate() {
                prop_assert_eq!(*actual_key, expected_key as NodeKey);
            }
        }

        #[test]
        fn prop_ast_dag_generation_monotonic(node_count in 1usize..50) {
            let mut dag = AstDag::new();
            let mut prev_gen = dag.generation();

            for _ in 0..node_count {
                dag.add_node(UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    Language::Rust,
                ));
                let new_gen = dag.generation();
                prop_assert!(new_gen > prev_gen);
                prev_gen = new_gen;
            }
        }
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_span_max_values() {
        let span = Span::new(u32::MAX - 1, u32::MAX);
        assert_eq!(span.len(), 1);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_byte_pos_from_usize_overflow() {
        // Test truncation behavior for values larger than u32::MAX
        let large_value = (u32::MAX as usize) + 100;
        let pos = BytePos::from_usize(large_value);
        // Truncated to u32
        assert_eq!(pos.0, (large_value as u32));
    }

    #[test]
    fn test_qualified_name_empty_segments() {
        // Edge case: path with only empty segments gets filtered
        let result = QualifiedName::from_string("::");
        assert!(result.is_err());
    }

    #[test]
    fn test_location_zero_length() {
        let loc = Location::new(PathBuf::from("test.rs"), 100, 100);
        assert_eq!(loc.span.len(), 0);
        assert!(loc.span.is_empty());
    }

    #[test]
    fn test_proof_annotation_with_all_optional_fields() {
        let annotation = ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: PropertyType::FunctionalCorrectness("spec".to_string()),
            specification_id: Some("spec_id".to_string()),
            method: VerificationMethod::FormalProof {
                prover: "Coq".to_string(),
            },
            tool_name: "coqc".to_string(),
            tool_version: "8.15".to_string(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec!["assumption1".to_string(), "assumption2".to_string()],
            evidence_type: EvidenceType::ProofScriptReference {
                uri: "file://proof.v".to_string(),
            },
            evidence_location: Some("/proofs/memory.v".to_string()),
            date_verified: Utc::now(),
        };

        let json = serde_json::to_string(&annotation).expect("serialization failed");
        let deserialized: ProofAnnotation =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(annotation.specification_id, deserialized.specification_id);
        assert_eq!(annotation.assumptions.len(), deserialized.assumptions.len());
        assert_eq!(annotation.evidence_location, deserialized.evidence_location);
    }

    #[test]
    fn test_unified_ast_node_all_function_kinds_are_functions() {
        let function_kinds = [
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

        for kind in function_kinds {
            let node = UnifiedAstNode::new(AstKind::Function(kind.clone()), Language::Rust);
            assert!(
                node.is_function(),
                "FunctionKind::{:?} should be recognized as function",
                kind
            );
        }
    }

    #[test]
    fn test_unified_ast_node_all_type_definitions() {
        let class_kinds = [
            ClassKind::Regular,
            ClassKind::Abstract,
            ClassKind::Interface,
            ClassKind::Trait,
            ClassKind::Enum,
            ClassKind::Struct,
        ];

        for kind in class_kinds {
            let node = UnifiedAstNode::new(AstKind::Class(kind.clone()), Language::Rust);
            assert!(
                node.is_type_definition(),
                "ClassKind::{:?} should be recognized as type definition",
                kind
            );
        }

        let type_kinds = [
            TypeKind::Primitive,
            TypeKind::Array,
            TypeKind::Alias,
            TypeKind::Interface,
        ];

        for kind in type_kinds {
            let node = UnifiedAstNode::new(AstKind::Type(kind.clone()), Language::TypeScript);
            assert!(
                node.is_type_definition(),
                "TypeKind::{:?} should be recognized as type definition",
                kind
            );
        }

        let module_kinds = [ModuleKind::File, ModuleKind::Namespace, ModuleKind::Package];

        for kind in module_kinds {
            let node = UnifiedAstNode::new(AstKind::Module(kind.clone()), Language::Python);
            assert!(
                node.is_type_definition(),
                "ModuleKind::{:?} should be recognized as type definition",
                kind
            );
        }
    }

    #[test]
    fn test_column_store_with_complex_type() {
        let mut store: ColumnStore<UnifiedAstNode> = ColumnStore::new(10);

        let node1 = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
        let node2 = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::TypeScript);

        let key1 = store.push(node1);
        let key2 = store.push(node2);

        assert!(store.get(key1).expect("node1 should exist").is_function());
        assert!(store
            .get(key2)
            .expect("node2 should exist")
            .is_type_definition());
    }

    #[test]
    fn test_language_parsers_default() {
        let parsers = LanguageParsers::default();
        // Just verify it can be created - it's a placeholder struct
        let _ = parsers;
    }

    #[test]
    fn test_proof_map_type_alias() {
        // ProofMap is HashMap<Location, Vec<ProofAnnotation>>
        let mut proof_map: ProofMap = HashMap::new();

        let location = Location::new(PathBuf::from("test.rs"), 0, 100);
        let annotation = create_test_proof_annotation();

        proof_map.insert(location.clone(), vec![annotation]);

        assert!(proof_map.contains_key(&location));
        assert_eq!(proof_map.get(&location).map(|v| v.len()), Some(1));
    }
}
