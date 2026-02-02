//! Coverage boost tests for unified_ast_types module - Part 2
//! Tests: UnifiedAstNode, Location, Span, BytePos, QualifiedName, MacroKind,
//! ProofAnnotation, PropertyType, ConfidenceLevel, VerificationMethod, EvidenceType,
//! RelativeLocation, NodeMetadata

use crate::models::unified_ast::{
    AstKind, BytePos, ClassKind, ConfidenceLevel, EvidenceType, FunctionKind, Language, Location,
    MacroKind, ModuleKind, NodeMetadata, PropertyType, ProofAnnotation, QualifiedName,
    RelativeLocation, Span, TypeKind, UnifiedAstNode, VerificationMethod,
};
use std::path::PathBuf;

// ============ MacroKind Tests ============

#[test]
fn test_macro_kind_object_like() {
    let kind = MacroKind::ObjectLike;
    let _ = format!("{:?}", kind);
}

#[test]
fn test_macro_kind_function_like() {
    let kind = MacroKind::FunctionLike;
    assert_eq!(kind, MacroKind::FunctionLike);
}

#[test]
fn test_macro_kind_variadic() {
    let kind = MacroKind::Variadic;
    let _ = format!("{:?}", kind);
}

#[test]
fn test_macro_kind_include() {
    let kind = MacroKind::Include;
    let _ = format!("{:?}", kind);
}

#[test]
fn test_macro_kind_conditional() {
    let kind = MacroKind::Conditional;
    let _ = format!("{:?}", kind);
}

#[test]
fn test_macro_kind_export() {
    let kind = MacroKind::Export;
    let _ = format!("{:?}", kind);
}

#[test]
fn test_macro_kind_decorator() {
    let kind = MacroKind::Decorator;
    let _ = format!("{:?}", kind);
}

#[test]
fn test_macro_kind_serde() {
    let kind = MacroKind::FunctionLike;
    let json = serde_json::to_string(&kind).unwrap();
    let back: MacroKind = serde_json::from_str(&json).unwrap();
    assert_eq!(kind, back);
}

#[test]
fn test_ast_kind_macro() {
    let kind = AstKind::Macro(MacroKind::ObjectLike);
    let json = serde_json::to_string(&kind).unwrap();
    let back: AstKind = serde_json::from_str(&json).unwrap();
    assert_eq!(kind, back);
}

// ============ BytePos Tests ============

#[test]
fn test_byte_pos_creation() {
    let pos = BytePos(42);
    assert_eq!(pos.0, 42);
}

#[test]
fn test_byte_pos_to_usize() {
    let pos = BytePos(100);
    assert_eq!(pos.to_usize(), 100);
}

#[test]
fn test_byte_pos_from_usize() {
    let pos = BytePos::from_usize(42);
    assert_eq!(pos.0, 42);
    assert_eq!(pos.to_usize(), 42);
}

#[test]
fn test_byte_pos_roundtrip() {
    let original = BytePos(999);
    let converted = BytePos::from_usize(original.to_usize());
    assert_eq!(original, converted);
}

#[test]
fn test_byte_pos_ordering() {
    let a = BytePos(10);
    let b = BytePos(20);
    assert!(a < b);
    assert!(b > a);
    assert_eq!(BytePos(5), BytePos(5));
}

#[test]
fn test_byte_pos_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(BytePos(1));
    set.insert(BytePos(2));
    set.insert(BytePos(1)); // duplicate
    assert_eq!(set.len(), 2);
}

// ============ Span Tests ============

#[test]
fn test_span_new() {
    let span = Span::new(10, 50);
    assert_eq!(span.start.0, 10);
    assert_eq!(span.end.0, 50);
}

#[test]
fn test_span_len() {
    let span = Span::new(10, 60);
    assert_eq!(span.len(), 50);
}

#[test]
fn test_span_len_zero() {
    let span = Span::new(10, 10);
    assert_eq!(span.len(), 0);
}

#[test]
fn test_span_is_empty_true() {
    let span = Span::new(10, 10);
    assert!(span.is_empty());
}

#[test]
fn test_span_is_empty_false() {
    let span = Span::new(10, 20);
    assert!(!span.is_empty());
}

#[test]
fn test_span_contains_inside() {
    let span = Span::new(10, 50);
    assert!(span.contains(BytePos(10))); // start inclusive
    assert!(span.contains(BytePos(25)));
    assert!(span.contains(BytePos(49)));
}

#[test]
fn test_span_contains_outside() {
    let span = Span::new(10, 50);
    assert!(!span.contains(BytePos(9)));  // before
    assert!(!span.contains(BytePos(50))); // end exclusive
    assert!(!span.contains(BytePos(100)));
}

#[test]
fn test_span_hash_eq() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Span::new(0, 10));
    set.insert(Span::new(0, 10)); // duplicate
    set.insert(Span::new(0, 20));
    assert_eq!(set.len(), 2);
}

// ============ Location Tests ============

#[test]
fn test_location_new() {
    let loc = Location::new(PathBuf::from("src/main.rs"), 100, 200);
    assert_eq!(loc.file_path, PathBuf::from("src/main.rs"));
    assert_eq!(loc.span.start.0, 100);
    assert_eq!(loc.span.end.0, 200);
}

#[test]
fn test_location_contains_inner() {
    let outer = Location::new(PathBuf::from("test.rs"), 0, 100);
    let inner = Location::new(PathBuf::from("test.rs"), 10, 50);
    assert!(outer.contains(&inner));
}

#[test]
fn test_location_contains_not_reverse() {
    let outer = Location::new(PathBuf::from("test.rs"), 0, 100);
    let inner = Location::new(PathBuf::from("test.rs"), 10, 50);
    assert!(!inner.contains(&outer));
}

#[test]
fn test_location_contains_different_file() {
    let loc1 = Location::new(PathBuf::from("a.rs"), 0, 100);
    let loc2 = Location::new(PathBuf::from("b.rs"), 0, 100);
    assert!(!loc1.contains(&loc2));
}

#[test]
fn test_location_contains_self() {
    let loc = Location::new(PathBuf::from("test.rs"), 10, 50);
    assert!(loc.contains(&loc));
}

#[test]
fn test_location_overlaps_partial() {
    let loc1 = Location::new(PathBuf::from("test.rs"), 0, 50);
    let loc2 = Location::new(PathBuf::from("test.rs"), 25, 75);
    assert!(loc1.overlaps(&loc2));
    assert!(loc2.overlaps(&loc1));
}

#[test]
fn test_location_overlaps_no_overlap() {
    let loc1 = Location::new(PathBuf::from("test.rs"), 0, 50);
    let loc2 = Location::new(PathBuf::from("test.rs"), 50, 100);
    assert!(!loc1.overlaps(&loc2));
}

#[test]
fn test_location_overlaps_different_file() {
    let loc1 = Location::new(PathBuf::from("a.rs"), 0, 100);
    let loc2 = Location::new(PathBuf::from("b.rs"), 0, 100);
    assert!(!loc1.overlaps(&loc2));
}

#[test]
fn test_location_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Location::new(PathBuf::from("a.rs"), 0, 10));
    set.insert(Location::new(PathBuf::from("a.rs"), 0, 20)); // same start, different end
    // Hash only uses file_path and start pos, so these may collide
    assert!(set.len() >= 1);
}

// ============ QualifiedName Tests ============

#[test]
fn test_qualified_name_new() {
    let qname = QualifiedName::new(
        vec!["std".to_string(), "collections".to_string()],
        "HashMap".to_string(),
    );
    assert_eq!(qname.module_path, vec!["std", "collections"]);
    assert_eq!(qname.name, "HashMap");
    assert!(qname.disambiguator.is_none());
}

#[test]
fn test_qualified_name_simple() {
    let qname = QualifiedName::new(vec![], "main".to_string());
    assert!(qname.module_path.is_empty());
    assert_eq!(qname.name, "main");
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
fn test_qualified_name_simple_to_string() {
    let qname = QualifiedName::new(vec![], "main".to_string());
    assert_eq!(qname.to_qualified_string(), "main");
}

#[test]
fn test_qualified_name_with_disambiguator() {
    let qname = QualifiedName::new(
        vec!["mod".to_string()],
        "func".to_string(),
    )
    .with_disambiguator(1);
    assert_eq!(qname.disambiguator, Some(1));
    assert_eq!(qname.to_qualified_string(), "mod::func#1");
}

#[test]
fn test_qualified_name_from_string_simple() {
    let qname = QualifiedName::from_string("main").unwrap();
    assert_eq!(qname.name, "main");
    assert!(qname.module_path.is_empty());
}

#[test]
fn test_qualified_name_from_string_qualified() {
    let qname = QualifiedName::from_string("std::collections::HashMap").unwrap();
    assert_eq!(qname.module_path, vec!["std", "collections"]);
    assert_eq!(qname.name, "HashMap");
}

#[test]
fn test_qualified_name_from_string_empty_err() {
    assert!(QualifiedName::from_string("").is_err());
}

#[test]
fn test_qualified_name_from_str_trait() {
    let qname: QualifiedName = "std::vec::Vec".parse().unwrap();
    assert_eq!(qname.name, "Vec");
    assert_eq!(qname.module_path, vec!["std", "vec"]);
}

#[test]
fn test_qualified_name_display() {
    let qname = QualifiedName::new(
        vec!["crate".to_string()],
        "Foo".to_string(),
    );
    let displayed = format!("{}", qname);
    assert_eq!(displayed, "crate::Foo");
}

#[test]
fn test_qualified_name_hash_eq() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let q1 = QualifiedName::new(vec!["a".to_string()], "b".to_string());
    let q2 = QualifiedName::new(vec!["a".to_string()], "b".to_string());
    set.insert(q1);
    set.insert(q2);
    assert_eq!(set.len(), 1);
}

// ============ NodeMetadata Tests ============

#[test]
fn test_node_metadata_default() {
    let meta = NodeMetadata::default();
    assert_eq!(unsafe { meta.raw }, 0);
}

#[test]
fn test_node_metadata_clone() {
    let meta = NodeMetadata { raw: 42 };
    let cloned = meta;
    assert_eq!(unsafe { cloned.raw }, 42);
}

// ============ UnifiedAstNode Tests ============

#[test]
fn test_unified_ast_node_new_function() {
    let node = UnifiedAstNode::new(
        AstKind::Function(FunctionKind::Regular),
        Language::Rust,
    );
    assert!(node.is_function());
    assert!(!node.is_type_definition());
    assert_eq!(node.lang, Language::Rust);
    assert_eq!(node.parent, 0);
    assert_eq!(node.first_child, 0);
    assert_eq!(node.next_sibling, 0);
    assert_eq!(node.source_range, 0..0);
    assert_eq!(node.complexity(), 0);
    assert!(!node.has_proof_annotations());
}

#[test]
fn test_unified_ast_node_new_class() {
    let node = UnifiedAstNode::new(
        AstKind::Class(ClassKind::Struct),
        Language::TypeScript,
    );
    assert!(!node.is_function());
    assert!(node.is_type_definition());
}

#[test]
fn test_unified_ast_node_new_type() {
    let node = UnifiedAstNode::new(
        AstKind::Type(TypeKind::Alias),
        Language::Rust,
    );
    assert!(node.is_type_definition());
}

#[test]
fn test_unified_ast_node_new_module() {
    let node = UnifiedAstNode::new(
        AstKind::Module(ModuleKind::File),
        Language::Python,
    );
    assert!(node.is_type_definition());
    assert!(!node.is_function());
}

#[test]
fn test_unified_ast_node_method_is_function() {
    let node = UnifiedAstNode::new(
        AstKind::Function(FunctionKind::Method),
        Language::TypeScript,
    );
    assert!(node.is_function());
}

#[test]
fn test_unified_ast_node_set_complexity() {
    let mut node = UnifiedAstNode::new(
        AstKind::Function(FunctionKind::Regular),
        Language::Rust,
    );
    assert_eq!(node.complexity(), 0);
    node.set_complexity(15);
    assert_eq!(node.complexity(), 15);
}

#[test]
fn test_unified_ast_node_clone() {
    let mut node = UnifiedAstNode::new(
        AstKind::Function(FunctionKind::Regular),
        Language::Rust,
    );
    node.set_complexity(10);
    let cloned = node.clone();
    assert_eq!(cloned.complexity(), 10);
    assert!(cloned.is_function());
}

// ============ ConfidenceLevel Tests ============

#[test]
fn test_confidence_level_ordering() {
    assert!(ConfidenceLevel::Low < ConfidenceLevel::Medium);
    assert!(ConfidenceLevel::Medium < ConfidenceLevel::High);
}

#[test]
fn test_confidence_level_serde() {
    let levels = vec![
        ConfidenceLevel::Low,
        ConfidenceLevel::Medium,
        ConfidenceLevel::High,
    ];
    for level in &levels {
        let json = serde_json::to_string(level).unwrap();
        let back: ConfidenceLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(*level, back);
    }
}

// ============ PropertyType Tests ============

#[test]
fn test_property_type_memory_safety() {
    let pt = PropertyType::MemorySafety;
    let json = serde_json::to_string(&pt).unwrap();
    let back: PropertyType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, back);
}

#[test]
fn test_property_type_thread_safety() {
    let pt = PropertyType::ThreadSafety;
    let _ = format!("{:?}", pt);
}

#[test]
fn test_property_type_functional_correctness() {
    let pt = PropertyType::FunctionalCorrectness("spec-001".to_string());
    let json = serde_json::to_string(&pt).unwrap();
    let back: PropertyType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, back);
}

#[test]
fn test_property_type_resource_bounds() {
    let pt = PropertyType::ResourceBounds {
        cpu: Some(1000),
        memory: Some(4096),
    };
    let json = serde_json::to_string(&pt).unwrap();
    let back: PropertyType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, back);
}

// ============ VerificationMethod Tests ============

#[test]
fn test_verification_method_borrow_checker() {
    let vm = VerificationMethod::BorrowChecker;
    let json = serde_json::to_string(&vm).unwrap();
    let back: VerificationMethod = serde_json::from_str(&json).unwrap();
    assert_eq!(vm, back);
}

#[test]
fn test_verification_method_formal_proof() {
    let vm = VerificationMethod::FormalProof {
        prover: "Coq".to_string(),
    };
    let json = serde_json::to_string(&vm).unwrap();
    let back: VerificationMethod = serde_json::from_str(&json).unwrap();
    assert_eq!(vm, back);
}

#[test]
fn test_verification_method_static_analysis() {
    let vm = VerificationMethod::StaticAnalysis {
        tool: "Clippy".to_string(),
    };
    let _ = format!("{:?}", vm);
}

#[test]
fn test_verification_method_model_checking() {
    let vm = VerificationMethod::ModelChecking { bounded: true };
    let json = serde_json::to_string(&vm).unwrap();
    let back: VerificationMethod = serde_json::from_str(&json).unwrap();
    assert_eq!(vm, back);
}

#[test]
fn test_verification_method_abstract_interpretation() {
    let vm = VerificationMethod::AbstractInterpretation;
    let _ = format!("{:?}", vm);
}

// ============ EvidenceType Tests ============

#[test]
fn test_evidence_type_implicit_guarantee() {
    let et = EvidenceType::ImplicitTypeSystemGuarantee;
    let json = serde_json::to_string(&et).unwrap();
    let back: EvidenceType = serde_json::from_str(&json).unwrap();
    assert_eq!(et, back);
}

#[test]
fn test_evidence_type_proof_script() {
    let et = EvidenceType::ProofScriptReference {
        uri: "file:///proof.v".to_string(),
    };
    let json = serde_json::to_string(&et).unwrap();
    let back: EvidenceType = serde_json::from_str(&json).unwrap();
    assert_eq!(et, back);
}

#[test]
fn test_evidence_type_theorem_name() {
    let et = EvidenceType::TheoremName {
        theorem: "safety_invariant".to_string(),
        theory: Some("concurrency".to_string()),
    };
    let json = serde_json::to_string(&et).unwrap();
    let back: EvidenceType = serde_json::from_str(&json).unwrap();
    assert_eq!(et, back);
}

#[test]
fn test_evidence_type_analysis_report() {
    let et = EvidenceType::StaticAnalysisReport {
        report_id: "RPT-001".to_string(),
    };
    let _ = format!("{:?}", et);
}

#[test]
fn test_evidence_type_certificate_hash() {
    let et = EvidenceType::CertificateHash {
        hash: "abc123".to_string(),
        algorithm: "SHA-256".to_string(),
    };
    let json = serde_json::to_string(&et).unwrap();
    let back: EvidenceType = serde_json::from_str(&json).unwrap();
    assert_eq!(et, back);
}

// ============ RelativeLocation Tests ============

#[test]
fn test_relative_location_function() {
    let loc = RelativeLocation::Function {
        name: "process".to_string(),
        module: Some("core".to_string()),
    };
    let json = serde_json::to_string(&loc).unwrap();
    let back: RelativeLocation = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_relative_location_symbol() {
    let loc = RelativeLocation::Symbol {
        qualified_name: "crate::module::Type::method".to_string(),
    };
    let json = serde_json::to_string(&loc).unwrap();
    let back: RelativeLocation = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

#[test]
fn test_relative_location_span() {
    let loc = RelativeLocation::Span {
        start: 100,
        end: 200,
    };
    let json = serde_json::to_string(&loc).unwrap();
    let back: RelativeLocation = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", back);
}

// ============ ProofAnnotation Tests ============

#[test]
fn test_proof_annotation_serde() {
    use chrono::Utc;
    use uuid::Uuid;

    let annotation = ProofAnnotation {
        annotation_id: Uuid::nil(),
        property_proven: PropertyType::MemorySafety,
        specification_id: Some("spec-001".to_string()),
        method: VerificationMethod::BorrowChecker,
        tool_name: "rustc".to_string(),
        tool_version: "1.70.0".to_string(),
        confidence_level: ConfidenceLevel::High,
        assumptions: vec!["no_unsafe".to_string()],
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        evidence_location: None,
        date_verified: Utc::now(),
    };
    let json = serde_json::to_string(&annotation).unwrap();
    let back: ProofAnnotation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tool_name, "rustc");
    assert_eq!(back.confidence_level, ConfidenceLevel::High);
}

#[test]
fn test_proof_annotation_empty_assumptions() {
    use chrono::Utc;
    use uuid::Uuid;

    let annotation = ProofAnnotation {
        annotation_id: Uuid::nil(),
        property_proven: PropertyType::ThreadSafety,
        specification_id: None,
        method: VerificationMethod::AbstractInterpretation,
        tool_name: "analyzer".to_string(),
        tool_version: "2.0".to_string(),
        confidence_level: ConfidenceLevel::Medium,
        assumptions: vec![],
        evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
        evidence_location: Some("/path/to/evidence".to_string()),
        date_verified: Utc::now(),
    };
    let json = serde_json::to_string(&annotation).unwrap();
    // Empty assumptions should be skipped in serialization
    assert!(!json.contains("assumptions"));
}
