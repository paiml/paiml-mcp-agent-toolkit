#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for unified_ast_types module - Part 3
//! Tests: AstDag, ColumnStore, UnifiedAstNode advanced methods, LanguageParsers,
//! C/C++ specific flags, additional PropertyType variants, serde round-trips

use crate::models::unified_ast::{
    AstDag, AstKind, BytePos, ClassKind, ColumnStore, ConfidenceLevel, EvidenceType, FunctionKind,
    ImportKind, Language, Location, MacroKind, ModuleKind, NodeFlags, NodeMetadata,
    ProofAnnotation, PropertyType, QualifiedName, RelativeLocation, Span, StmtKind, TypeKind,
    UnifiedAstNode, VarKind, VerificationMethod,
};
use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

// ============ ColumnStore Tests ============

#[test]
fn test_column_store_new() {
    let store: ColumnStore<u32> = ColumnStore::new(100);
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn test_column_store_push() {
    let mut store: ColumnStore<u32> = ColumnStore::new(10);
    let key1 = store.push(42);
    let key2 = store.push(100);
    assert_eq!(key1, 0);
    assert_eq!(key2, 1);
    assert_eq!(store.len(), 2);
}

#[test]
fn test_column_store_get() {
    let mut store: ColumnStore<String> = ColumnStore::new(10);
    store.push("hello".to_string());
    store.push("world".to_string());

    assert_eq!(store.get(0), Some(&"hello".to_string()));
    assert_eq!(store.get(1), Some(&"world".to_string()));
    assert_eq!(store.get(2), None);
}

#[test]
fn test_column_store_get_mut() {
    let mut store: ColumnStore<i32> = ColumnStore::new(10);
    store.push(10);
    store.push(20);

    if let Some(val) = store.get_mut(0) {
        *val = 100;
    }
    assert_eq!(store.get(0), Some(&100));
}

#[test]
fn test_column_store_iter() {
    let mut store: ColumnStore<i32> = ColumnStore::new(10);
    store.push(1);
    store.push(2);
    store.push(3);

    let sum: i32 = store.iter().sum();
    assert_eq!(sum, 6);
}

#[test]
fn test_column_store_is_empty() {
    let mut store: ColumnStore<u32> = ColumnStore::new(10);
    assert!(store.is_empty());
    store.push(1);
    assert!(!store.is_empty());
}

#[test]
fn test_column_store_with_ast_nodes() {
    let mut store: ColumnStore<UnifiedAstNode> = ColumnStore::new(100);
    let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    let key = store.push(node);

    assert_eq!(key, 0);
    let retrieved = store.get(key).unwrap();
    assert!(retrieved.is_function());
}

// ============ AstDag Tests ============

#[test]
fn test_ast_dag_new() {
    let dag = AstDag::new();
    assert!(dag.nodes.is_empty());
    assert_eq!(dag.generation(), 0);
}

#[test]
fn test_ast_dag_default() {
    let dag = AstDag::default();
    assert!(dag.nodes.is_empty());
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
fn test_ast_dag_add_multiple_nodes() {
    let mut dag = AstDag::new();

    let func_node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    let class_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::TypeScript);
    let type_node = UnifiedAstNode::new(AstKind::Type(TypeKind::Alias), Language::Python);

    let key1 = dag.add_node(func_node);
    let key2 = dag.add_node(class_node);
    let key3 = dag.add_node(type_node);

    assert_eq!(key1, 0);
    assert_eq!(key2, 1);
    assert_eq!(key3, 2);
    assert_eq!(dag.nodes.len(), 3);
    assert_eq!(dag.generation(), 3);
}

#[test]
fn test_ast_dag_dirty_nodes() {
    let mut dag = AstDag::new();

    let node1 = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    let node2 = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);

    let key1 = dag.add_node(node1);
    let key2 = dag.add_node(node2);

    let dirty: Vec<u32> = dag.dirty_nodes().collect();
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
}

#[test]
fn test_ast_dag_mark_clean_preserves_node() {
    let mut dag = AstDag::new();

    let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    let key = dag.add_node(node);

    dag.mark_clean(key);

    // Node should still exist
    assert!(dag.nodes.get(key).is_some());
    assert!(dag.nodes.get(key).unwrap().is_function());
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
        Language::Rust,
    ));
    assert_eq!(dag.generation(), 2);

    // mark_clean does not increment generation
    dag.mark_clean(0);
    assert_eq!(dag.generation(), 2);
}

#[test]
fn test_ast_dag_parsers_default() {
    let dag = AstDag::new();
    // LanguageParsers is default-initialized
    let _ = &dag.parsers;
}

// ============ UnifiedAstNode Advanced Tests ============

#[test]
fn test_unified_ast_node_location() {
    let mut node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    node.source_range = 100..200;

    let path = PathBuf::from("src/lib.rs");
    let location = node.location(&path);

    assert_eq!(location.file_path, PathBuf::from("src/lib.rs"));
    assert_eq!(location.span.start.0, 100);
    assert_eq!(location.span.end.0, 200);
}

#[test]
fn test_unified_ast_node_add_proof_annotation() {
    let mut node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    assert!(!node.has_proof_annotations());
    assert_eq!(node.proof_annotations().len(), 0);

    let annotation = ProofAnnotation {
        annotation_id: Uuid::new_v4(),
        property_proven: PropertyType::MemorySafety,
        specification_id: None,
        method: VerificationMethod::BorrowChecker,
        tool_name: "rustc".to_string(),
        tool_version: "1.70.0".to_string(),
        confidence_level: ConfidenceLevel::High,
        assumptions: vec![],
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        evidence_location: None,
        date_verified: Utc::now(),
    };

    node.add_proof_annotation(annotation);

    assert!(node.has_proof_annotations());
    assert_eq!(node.proof_annotations().len(), 1);
}

#[test]
fn test_unified_ast_node_multiple_proof_annotations() {
    let mut node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

    let annotation1 = ProofAnnotation {
        annotation_id: Uuid::new_v4(),
        property_proven: PropertyType::MemorySafety,
        specification_id: None,
        method: VerificationMethod::BorrowChecker,
        tool_name: "rustc".to_string(),
        tool_version: "1.70.0".to_string(),
        confidence_level: ConfidenceLevel::High,
        assumptions: vec![],
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        evidence_location: None,
        date_verified: Utc::now(),
    };

    let annotation2 = ProofAnnotation {
        annotation_id: Uuid::new_v4(),
        property_proven: PropertyType::ThreadSafety,
        specification_id: Some("thread-spec".to_string()),
        method: VerificationMethod::StaticAnalysis {
            tool: "Clippy".to_string(),
        },
        tool_name: "clippy".to_string(),
        tool_version: "0.1.0".to_string(),
        confidence_level: ConfidenceLevel::Medium,
        assumptions: vec!["single-threaded".to_string()],
        evidence_type: EvidenceType::StaticAnalysisReport {
            report_id: "RPT-001".to_string(),
        },
        evidence_location: Some("/reports/thread.json".to_string()),
        date_verified: Utc::now(),
    };

    node.add_proof_annotation(annotation1);
    node.add_proof_annotation(annotation2);

    assert_eq!(node.proof_annotations().len(), 2);
}

#[test]
fn test_unified_ast_node_debug_format() {
    let mut node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    node.set_complexity(15);
    node.source_range = 0..100;
    node.semantic_hash = 12345;
    node.structural_hash = 67890;

    let debug_str = format!("{:?}", node);

    assert!(debug_str.contains("UnifiedAstNode"));
    assert!(debug_str.contains("Function"));
    assert!(debug_str.contains("Rust"));
}

#[test]
fn test_unified_ast_node_proof_annotations_empty() {
    let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    let annotations = node.proof_annotations();
    assert!(annotations.is_empty());
}

#[test]
fn test_unified_ast_node_has_proof_annotations_false() {
    let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    assert!(!node.has_proof_annotations());
}

#[test]
fn test_unified_ast_node_various_function_kinds() {
    let kinds = vec![
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

    for kind in kinds {
        let node = UnifiedAstNode::new(AstKind::Function(kind.clone()), Language::Rust);
        assert!(node.is_function());
        assert!(!node.is_type_definition());
    }
}

#[test]
fn test_unified_ast_node_various_class_kinds() {
    let kinds = vec![
        ClassKind::Regular,
        ClassKind::Abstract,
        ClassKind::Interface,
        ClassKind::Trait,
        ClassKind::Enum,
        ClassKind::Struct,
    ];

    for kind in kinds {
        let node = UnifiedAstNode::new(AstKind::Class(kind.clone()), Language::TypeScript);
        assert!(node.is_type_definition());
        assert!(!node.is_function());
    }
}

// ============ NodeFlags C-specific Tests ============

#[test]
fn test_node_flags_inline() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::INLINE);
    assert!(flags.has(NodeFlags::INLINE));
}

#[test]
fn test_node_flags_volatile() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::VOLATILE);
    assert!(flags.has(NodeFlags::VOLATILE));
}

#[test]
fn test_node_flags_restrict() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::RESTRICT);
    assert!(flags.has(NodeFlags::RESTRICT));
}

#[test]
fn test_node_flags_extern() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::EXTERN);
    assert!(flags.has(NodeFlags::EXTERN));
}

// ============ NodeFlags C++-specific Tests ============

#[test]
fn test_node_flags_virtual() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::VIRTUAL);
    assert!(flags.has(NodeFlags::VIRTUAL));
}

#[test]
fn test_node_flags_override() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::OVERRIDE);
    assert!(flags.has(NodeFlags::OVERRIDE));
}

#[test]
fn test_node_flags_final() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::FINAL);
    assert!(flags.has(NodeFlags::FINAL));
}

#[test]
fn test_node_flags_mutable() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::MUTABLE);
    assert!(flags.has(NodeFlags::MUTABLE));
}

#[test]
fn test_node_flags_constexpr() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::CONSTEXPR);
    assert!(flags.has(NodeFlags::CONSTEXPR));
}

#[test]
fn test_node_flags_noexcept() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::NOEXCEPT);
    assert!(flags.has(NodeFlags::NOEXCEPT));
}

#[test]
fn test_node_flags_debug() {
    let mut flags = NodeFlags::new();
    flags.set(NodeFlags::ASYNC);
    let debug_str = format!("{:?}", flags);
    assert!(debug_str.contains("NodeFlags"));
}

// ============ PropertyType Additional Variants ============

#[test]
fn test_property_type_data_race_freeze() {
    let pt = PropertyType::DataRaceFreeze;
    let json = serde_json::to_string(&pt).unwrap();
    let back: PropertyType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, back);
}

#[test]
fn test_property_type_termination() {
    let pt = PropertyType::Termination;
    let json = serde_json::to_string(&pt).unwrap();
    let back: PropertyType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, back);
}

#[test]
fn test_property_type_resource_bounds_partial() {
    let pt = PropertyType::ResourceBounds {
        cpu: Some(1000),
        memory: None,
    };
    let json = serde_json::to_string(&pt).unwrap();
    let back: PropertyType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, back);
}

#[test]
fn test_property_type_resource_bounds_none() {
    let pt = PropertyType::ResourceBounds {
        cpu: None,
        memory: None,
    };
    let json = serde_json::to_string(&pt).unwrap();
    let back: PropertyType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, back);
}

#[test]
fn test_property_type_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(PropertyType::MemorySafety);
    set.insert(PropertyType::ThreadSafety);
    set.insert(PropertyType::MemorySafety); // duplicate
    assert_eq!(set.len(), 2);
}

// ============ ModuleKind Package Variant ============

#[test]
fn test_module_kind_package() {
    let kind = ModuleKind::Package;
    let json = serde_json::to_string(&kind).unwrap();
    let back: ModuleKind = serde_json::from_str(&json).unwrap();
    assert_eq!(kind, back);
}

// ============ QualifiedName Edge Cases ============

#[test]
fn test_qualified_name_from_string_trailing_colon() {
    let result = QualifiedName::from_string("foo::bar::");
    // Should fail because last component is empty
    assert!(result.is_err());
}

#[test]
fn test_qualified_name_deep_nesting() {
    let qname = QualifiedName::new(
        vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ],
        "e".to_string(),
    );
    assert_eq!(qname.to_qualified_string(), "a::b::c::d::e");
}

#[test]
fn test_qualified_name_clone() {
    let qname = QualifiedName::new(vec!["mod".to_string()], "func".to_string());
    let cloned = qname.clone();
    assert_eq!(qname.name, cloned.name);
    assert_eq!(qname.module_path, cloned.module_path);
}

// ============ NodeMetadata Union Tests ============

#[test]
fn test_node_metadata_complexity_field() {
    let meta = NodeMetadata { complexity: 42 };
    assert_eq!(unsafe { meta.complexity }, 42);
}

#[test]
fn test_node_metadata_hash_field() {
    let meta = NodeMetadata { hash: 0xDEADBEEF };
    assert_eq!(unsafe { meta.hash }, 0xDEADBEEF);
}

#[test]
fn test_node_metadata_flags_field() {
    let meta = NodeMetadata { flags: 0xFF };
    assert_eq!(unsafe { meta.flags }, 0xFF);
}

#[test]
fn test_node_metadata_copy() {
    let meta = NodeMetadata { raw: 123 };
    let copied = meta;
    assert_eq!(unsafe { copied.raw }, 123);
}

// ============ Location Edge Cases ============

#[test]
fn test_location_span_len() {
    let loc = Location::new(PathBuf::from("test.rs"), 0, 100);
    assert_eq!(loc.span.len(), 100);
}

#[test]
fn test_location_debug() {
    let loc = Location::new(PathBuf::from("test.rs"), 10, 50);
    let debug_str = format!("{:?}", loc);
    assert!(debug_str.contains("Location"));
    assert!(debug_str.contains("test.rs"));
}

#[test]
fn test_location_clone() {
    let loc = Location::new(PathBuf::from("test.rs"), 10, 50);
    let cloned = loc.clone();
    assert_eq!(loc.file_path, cloned.file_path);
    assert_eq!(loc.span, cloned.span);
}

// ============ Span Edge Cases ============

#[test]
fn test_span_inverted_is_empty() {
    // When start >= end, is_empty returns true
    let span = Span::new(50, 10);
    assert!(span.is_empty());
}

#[test]
fn test_span_debug() {
    let span = Span::new(10, 50);
    let debug_str = format!("{:?}", span);
    assert!(debug_str.contains("Span"));
}

#[test]
fn test_span_clone() {
    let span = Span::new(10, 50);
    let cloned = span;
    assert_eq!(span, cloned);
}

// ============ BytePos Edge Cases ============

#[test]
fn test_byte_pos_zero() {
    let pos = BytePos(0);
    assert_eq!(pos.to_usize(), 0);
}

#[test]
fn test_byte_pos_max() {
    let pos = BytePos(u32::MAX);
    assert_eq!(pos.0, u32::MAX);
}

#[test]
fn test_byte_pos_debug() {
    let pos = BytePos(42);
    let debug_str = format!("{:?}", pos);
    assert!(debug_str.contains("BytePos"));
    assert!(debug_str.contains("42"));
}

// ============ AstKind Serde Round-trips ============

#[test]
fn test_ast_kind_function_all_variants_serde() {
    let variants = vec![
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

    for kind in variants {
        let json = serde_json::to_string(&kind).unwrap();
        let back: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back);
    }
}

#[test]
fn test_ast_kind_class_all_variants_serde() {
    let variants = vec![
        AstKind::Class(ClassKind::Regular),
        AstKind::Class(ClassKind::Abstract),
        AstKind::Class(ClassKind::Interface),
        AstKind::Class(ClassKind::Trait),
        AstKind::Class(ClassKind::Enum),
        AstKind::Class(ClassKind::Struct),
    ];

    for kind in variants {
        let json = serde_json::to_string(&kind).unwrap();
        let back: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back);
    }
}

#[test]
fn test_ast_kind_variable_all_variants_serde() {
    let variants = vec![
        AstKind::Variable(VarKind::Let),
        AstKind::Variable(VarKind::Const),
        AstKind::Variable(VarKind::Static),
        AstKind::Variable(VarKind::Field),
        AstKind::Variable(VarKind::Parameter),
    ];

    for kind in variants {
        let json = serde_json::to_string(&kind).unwrap();
        let back: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back);
    }
}

#[test]
fn test_ast_kind_import_all_variants_serde() {
    let variants = vec![
        AstKind::Import(ImportKind::Module),
        AstKind::Import(ImportKind::Named),
        AstKind::Import(ImportKind::Default),
        AstKind::Import(ImportKind::Namespace),
        AstKind::Import(ImportKind::Dynamic),
    ];

    for kind in variants {
        let json = serde_json::to_string(&kind).unwrap();
        let back: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back);
    }
}

#[test]
fn test_ast_kind_statement_all_variants_serde() {
    let variants = vec![
        AstKind::Statement(StmtKind::Block),
        AstKind::Statement(StmtKind::If),
        AstKind::Statement(StmtKind::For),
        AstKind::Statement(StmtKind::While),
        AstKind::Statement(StmtKind::Return),
        AstKind::Statement(StmtKind::Throw),
        AstKind::Statement(StmtKind::Try),
        AstKind::Statement(StmtKind::Switch),
        AstKind::Statement(StmtKind::Goto),
        AstKind::Statement(StmtKind::Label),
        AstKind::Statement(StmtKind::DoWhile),
        AstKind::Statement(StmtKind::ForEach),
        AstKind::Statement(StmtKind::Catch),
        AstKind::Statement(StmtKind::Break),
        AstKind::Statement(StmtKind::Continue),
        AstKind::Statement(StmtKind::Case),
    ];

    for kind in variants {
        let json = serde_json::to_string(&kind).unwrap();
        let back: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back);
    }
}

#[test]
fn test_ast_kind_type_all_variants_serde() {
    let variants = vec![
        AstKind::Type(TypeKind::Primitive),
        AstKind::Type(TypeKind::Array),
        AstKind::Type(TypeKind::Tuple),
        AstKind::Type(TypeKind::Union),
        AstKind::Type(TypeKind::Intersection),
        AstKind::Type(TypeKind::Generic),
        AstKind::Type(TypeKind::Function),
        AstKind::Type(TypeKind::Object),
        AstKind::Type(TypeKind::Pointer),
        AstKind::Type(TypeKind::Struct),
        AstKind::Type(TypeKind::Enum),
        AstKind::Type(TypeKind::Typedef),
        AstKind::Type(TypeKind::Class),
        AstKind::Type(TypeKind::Template),
        AstKind::Type(TypeKind::Namespace),
        AstKind::Type(TypeKind::Alias),
        AstKind::Type(TypeKind::Interface),
        AstKind::Type(TypeKind::Module),
        AstKind::Type(TypeKind::Annotation),
        AstKind::Type(TypeKind::Mapped),
        AstKind::Type(TypeKind::Conditional),
    ];

    for kind in variants {
        let json = serde_json::to_string(&kind).unwrap();
        let back: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back);
    }
}

#[test]
fn test_ast_kind_module_all_variants_serde() {
    let variants = vec![
        AstKind::Module(ModuleKind::File),
        AstKind::Module(ModuleKind::Namespace),
        AstKind::Module(ModuleKind::Package),
    ];

    for kind in variants {
        let json = serde_json::to_string(&kind).unwrap();
        let back: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back);
    }
}

#[test]
fn test_ast_kind_macro_all_variants_serde() {
    let variants = vec![
        AstKind::Macro(MacroKind::ObjectLike),
        AstKind::Macro(MacroKind::FunctionLike),
        AstKind::Macro(MacroKind::Variadic),
        AstKind::Macro(MacroKind::Include),
        AstKind::Macro(MacroKind::Conditional),
        AstKind::Macro(MacroKind::Export),
        AstKind::Macro(MacroKind::Decorator),
    ];

    for kind in variants {
        let json = serde_json::to_string(&kind).unwrap();
        let back: AstKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, back);
    }
}

// ============ Language Serde Round-trips ============

#[test]
fn test_language_all_variants_serde() {
    let variants = vec![
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

    for lang in variants {
        let json = serde_json::to_string(&lang).unwrap();
        let back: Language = serde_json::from_str(&json).unwrap();
        assert_eq!(lang, back);
    }
}

// ============ VerificationMethod All Variants ============

#[test]
fn test_verification_method_model_checking_unbounded() {
    let vm = VerificationMethod::ModelChecking { bounded: false };
    let json = serde_json::to_string(&vm).unwrap();
    let back: VerificationMethod = serde_json::from_str(&json).unwrap();
    assert_eq!(vm, back);
}

// ============ EvidenceType All Variants ============

#[test]
fn test_evidence_type_theorem_no_theory() {
    let et = EvidenceType::TheoremName {
        theorem: "soundness".to_string(),
        theory: None,
    };
    let json = serde_json::to_string(&et).unwrap();
    let back: EvidenceType = serde_json::from_str(&json).unwrap();
    assert_eq!(et, back);
}

// ============ RelativeLocation All Variants ============

#[test]
fn test_relative_location_function_no_module() {
    let loc = RelativeLocation::Function {
        name: "helper".to_string(),
        module: None,
    };
    let json = serde_json::to_string(&loc).unwrap();
    let back: RelativeLocation = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

// ============ ConfidenceLevel All Variants ============

#[test]
fn test_confidence_level_values() {
    assert_eq!(ConfidenceLevel::Low as u8, 1);
    assert_eq!(ConfidenceLevel::Medium as u8, 2);
    assert_eq!(ConfidenceLevel::High as u8, 3);
}

#[test]
fn test_confidence_level_equality() {
    assert_eq!(ConfidenceLevel::Low, ConfidenceLevel::Low);
    assert_ne!(ConfidenceLevel::Low, ConfidenceLevel::High);
}

#[test]
fn test_confidence_level_clone() {
    let level = ConfidenceLevel::Medium;
    let cloned = level;
    assert_eq!(level, cloned);
}

// ============ ProofAnnotation Advanced Tests ============

#[test]
fn test_proof_annotation_with_all_fields() {
    let annotation = ProofAnnotation {
        annotation_id: Uuid::new_v4(),
        property_proven: PropertyType::FunctionalCorrectness("spec-001".to_string()),
        specification_id: Some("formal-spec-v1".to_string()),
        method: VerificationMethod::FormalProof {
            prover: "Coq".to_string(),
        },
        tool_name: "coqc".to_string(),
        tool_version: "8.15.0".to_string(),
        confidence_level: ConfidenceLevel::High,
        assumptions: vec![
            "axiom1".to_string(),
            "axiom2".to_string(),
            "axiom3".to_string(),
        ],
        evidence_type: EvidenceType::ProofScriptReference {
            uri: "file:///proofs/main.v".to_string(),
        },
        evidence_location: Some("/var/proofs/main.v".to_string()),
        date_verified: Utc::now(),
    };

    let json = serde_json::to_string(&annotation).unwrap();
    let back: ProofAnnotation = serde_json::from_str(&json).unwrap();

    assert_eq!(back.tool_name, "coqc");
    assert_eq!(back.assumptions.len(), 3);
}

#[test]
fn test_proof_annotation_clone() {
    let annotation = ProofAnnotation {
        annotation_id: Uuid::nil(),
        property_proven: PropertyType::MemorySafety,
        specification_id: None,
        method: VerificationMethod::BorrowChecker,
        tool_name: "rustc".to_string(),
        tool_version: "1.70.0".to_string(),
        confidence_level: ConfidenceLevel::High,
        assumptions: vec![],
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        evidence_location: None,
        date_verified: Utc::now(),
    };

    let cloned = annotation.clone();
    assert_eq!(annotation.tool_name, cloned.tool_name);
    assert_eq!(annotation.confidence_level, cloned.confidence_level);
}

#[test]
fn test_proof_annotation_debug() {
    let annotation = ProofAnnotation {
        annotation_id: Uuid::nil(),
        property_proven: PropertyType::MemorySafety,
        specification_id: None,
        method: VerificationMethod::BorrowChecker,
        tool_name: "rustc".to_string(),
        tool_version: "1.70.0".to_string(),
        confidence_level: ConfidenceLevel::High,
        assumptions: vec![],
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        evidence_location: None,
        date_verified: Utc::now(),
    };

    let debug_str = format!("{:?}", annotation);
    assert!(debug_str.contains("ProofAnnotation"));
    assert!(debug_str.contains("rustc"));
}

// ============ Integration Tests ============

#[test]
fn test_full_dag_workflow() {
    let mut dag = AstDag::new();

    // Add function node
    let mut func = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);
    func.source_range = 0..100;
    func.set_complexity(5);
    let func_key = dag.add_node(func);

    // Add class node
    let mut class = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);
    class.source_range = 100..200;
    let class_key = dag.add_node(class);

    // Verify state
    assert_eq!(dag.nodes.len(), 2);
    assert_eq!(dag.generation(), 2);
    assert_eq!(dag.dirty_nodes().count(), 2);

    // Process nodes
    dag.mark_clean(func_key);
    assert_eq!(dag.dirty_nodes().count(), 1);

    dag.mark_clean(class_key);
    assert_eq!(dag.dirty_nodes().count(), 0);

    // Verify nodes still accessible
    let retrieved_func = dag.nodes.get(func_key).unwrap();
    assert!(retrieved_func.is_function());
    assert_eq!(retrieved_func.complexity(), 5);

    let retrieved_class = dag.nodes.get(class_key).unwrap();
    assert!(retrieved_class.is_type_definition());
}

#[test]
fn test_node_with_proof_annotations_in_dag() {
    let mut dag = AstDag::new();

    let mut node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

    let annotation = ProofAnnotation {
        annotation_id: Uuid::new_v4(),
        property_proven: PropertyType::MemorySafety,
        specification_id: None,
        method: VerificationMethod::BorrowChecker,
        tool_name: "rustc".to_string(),
        tool_version: "1.70.0".to_string(),
        confidence_level: ConfidenceLevel::High,
        assumptions: vec![],
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        evidence_location: None,
        date_verified: Utc::now(),
    };

    node.add_proof_annotation(annotation);
    let key = dag.add_node(node);

    let retrieved = dag.nodes.get(key).unwrap();
    assert!(retrieved.has_proof_annotations());
    assert_eq!(retrieved.proof_annotations().len(), 1);
}

#[test]
fn test_location_with_node() {
    let mut node = UnifiedAstNode::new(
        AstKind::Function(FunctionKind::Method),
        Language::TypeScript,
    );
    node.source_range = 500..750;

    let path = PathBuf::from("src/components/Button.tsx");
    let location = node.location(&path);

    assert_eq!(location.file_path, path);
    assert_eq!(location.span.len(), 250);
    assert!(location.span.contains(BytePos(600)));
    assert!(!location.span.contains(BytePos(800)));
}
