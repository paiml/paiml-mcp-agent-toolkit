#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified AST node representation and metadata.

use std::ops::Range;
use std::path::Path;

use super::kinds::AstKind;
use super::location::{BytePos, Location, Span};
use super::proof::ProofAnnotation;
use super::types::{Language, NodeFlags, NodeKey};

/// Node metadata union for language-specific data
#[repr(C)]
pub union NodeMetadata {
    pub complexity: u64,
    pub hash: u64,
    pub flags: u64,
    pub raw: u64,
}

// Safe default for union
impl Default for NodeMetadata {
    fn default() -> Self {
        Self { raw: 0 }
    }
}

// Manual Clone implementation for union
impl Clone for NodeMetadata {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for NodeMetadata {}

/// Unified AST node representation
///
/// This structure is carefully designed to be:
/// - Cache-line aligned (64 bytes + proof annotations)
/// - SIMD-friendly for vectorized operations
/// - Memory efficient with bit-packed fields
/// - Enhanced with formal verification metadata
#[repr(C, align(32))]
#[derive(Clone)]
pub struct UnifiedAstNode {
    // Core node data - 32 bytes aligned
    pub kind: AstKind,            // 2 bytes - language-agnostic
    pub lang: Language,           // 1 byte
    pub flags: NodeFlags,         // 1 byte
    pub parent: NodeKey,          // 4 bytes
    pub first_child: NodeKey,     // 4 bytes
    pub next_sibling: NodeKey,    // 4 bytes
    pub source_range: Range<u32>, // 8 bytes

    // Semantic data - 32 bytes
    pub semantic_hash: u64,     // 8 bytes - content hash
    pub structural_hash: u64,   // 8 bytes - structure hash
    pub name_vector: u64,       // 8 bytes - packed name embedding
    pub metadata: NodeMetadata, // 8 bytes - union type

    // Proof annotations - sparse allocation for performance
    pub proof_annotations: Option<Vec<ProofAnnotation>>,
}

impl UnifiedAstNode {
    /// Creates a new unified AST node with the specified kind and language.
    ///
    /// Initializes all fields to default values. The node is created with:
    /// - No parent, children, or siblings (all keys set to 0)
    /// - Empty source range (0..0)
    /// - Zero semantic and structural hashes
    /// - Default flags and metadata
    /// - No proof annotations
    ///
    /// # Performance
    ///
    /// - Memory: 64-128 bytes (cache-line aligned)
    /// - Time: O(1) initialization
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     UnifiedAstNode, AstKind, FunctionKind, Language, ClassKind
    /// };
    ///
    /// let func_node = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    ///
    /// assert!(func_node.is_function());
    /// assert_eq!(func_node.lang, Language::Rust);
    /// assert_eq!(func_node.parent, 0);
    /// assert_eq!(func_node.source_range, 0..0);
    /// assert!(!func_node.has_proof_annotations());
    ///
    /// let class_node = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Struct),
    ///     Language::TypeScript
    /// );
    ///
    /// assert!(class_node.is_type_definition());
    /// assert!(!class_node.is_function());
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(kind: AstKind, lang: Language) -> Self {
        Self {
            kind,
            lang,
            flags: NodeFlags::default(),
            parent: 0,
            first_child: 0,
            next_sibling: 0,
            source_range: 0..0,
            semantic_hash: 0,
            structural_hash: 0,
            name_vector: 0,
            metadata: NodeMetadata::default(),
            proof_annotations: None,
        }
    }

    /// Checks if this node represents a function-like construct.
    ///
    /// Returns true for any node with `AstKind::Function`, regardless
    /// of the specific function kind (regular, method, constructor, etc.).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     UnifiedAstNode, AstKind, FunctionKind, ClassKind, Language
    /// };
    ///
    /// let function = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    /// assert!(function.is_function());
    ///
    /// let method = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Method),
    ///     Language::TypeScript
    /// );
    /// assert!(method.is_function());
    ///
    /// let class = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Struct),
    ///     Language::Rust
    /// );
    /// assert!(!class.is_function());
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_function(&self) -> bool {
        matches!(self.kind, AstKind::Function(_))
    }

    /// Checks if this node represents a type definition.
    ///
    /// Returns true for classes, type aliases, modules, and other
    /// type-defining constructs across all supported languages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     UnifiedAstNode, AstKind, ClassKind, TypeKind,
    ///     ModuleKind, FunctionKind, Language
    /// };
    ///
    /// let struct_node = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Struct),
    ///     Language::Rust
    /// );
    /// assert!(struct_node.is_type_definition());
    ///
    /// let interface = UnifiedAstNode::new(
    ///     AstKind::Class(ClassKind::Interface),
    ///     Language::TypeScript
    /// );
    /// assert!(interface.is_type_definition());
    ///
    /// let type_alias = UnifiedAstNode::new(
    ///     AstKind::Type(TypeKind::Alias),
    ///     Language::Rust
    /// );
    /// assert!(type_alias.is_type_definition());
    ///
    /// let module = UnifiedAstNode::new(
    ///     AstKind::Module(ModuleKind::File),
    ///     Language::Python
    /// );
    /// assert!(module.is_type_definition());
    ///
    /// let function = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    /// assert!(!function.is_type_definition());
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_type_definition(&self) -> bool {
        matches!(
            self.kind,
            AstKind::Class(_) | AstKind::Type(_) | AstKind::Module(_)
        )
    }

    /// Get the complexity score for this node
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn complexity(&self) -> u32 {
        // SAFETY: Accessing the complexity field of the metadata union.
        // This is safe because:
        // 1. The metadata union is properly initialized in all constructors
        // 2. We only read the lower 32 bits which are always valid u32 values
        // 3. The bit mask (0xFFFFFFFF) ensures we only access the complexity portion
        unsafe { (self.metadata.complexity & 0xFFFFFFFF) as u32 }
    }

    /// Set the complexity score for this node
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_complexity(&mut self, complexity: u32) {
        self.metadata.complexity = u64::from(complexity);
    }

    /// Adds a formal verification proof annotation to this node.
    ///
    /// Proof annotations provide metadata about formally verified properties
    /// of the code represented by this AST node. Multiple annotations can
    /// be added to track different verified properties.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::unified_ast::{
    ///     UnifiedAstNode, AstKind, FunctionKind, Language,
    ///     ProofAnnotation, PropertyType, VerificationMethod,
    ///     ConfidenceLevel, EvidenceType
    /// };
    /// use uuid::Uuid;
    /// use chrono::Utc;
    ///
    /// let mut node = UnifiedAstNode::new(
    ///     AstKind::Function(FunctionKind::Regular),
    ///     Language::Rust
    /// );
    ///
    /// assert!(!node.has_proof_annotations());
    ///
    /// let annotation = ProofAnnotation {
    ///     annotation_id: Uuid::new_v4(),
    ///     property_proven: PropertyType::MemorySafety,
    ///     specification_id: Some("memory_safety_spec_v1".to_string()),
    ///     method: VerificationMethod::BorrowChecker,
    ///     tool_name: "rustc".to_string(),
    ///     tool_version: "1.70.0".to_string(),
    ///     confidence_level: ConfidenceLevel::High,
    ///     assumptions: vec![],
    ///     evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
    ///     evidence_location: None,
    ///     date_verified: Utc::now(),
    /// };
    ///
    /// node.add_proof_annotation(annotation);
    ///
    /// assert!(node.has_proof_annotations());
    /// assert_eq!(node.proof_annotations().len(), 1);
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_proof_annotation(&mut self, annotation: ProofAnnotation) {
        match &mut self.proof_annotations {
            Some(annotations) => annotations.push(annotation),
            None => self.proof_annotations = Some(vec![annotation]),
        }
    }

    /// Get all proof annotations for this node
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn proof_annotations(&self) -> &[ProofAnnotation] {
        self.proof_annotations.as_deref().unwrap_or(&[])
    }

    /// Check if this node has proof annotations
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_proof_annotations(&self) -> bool {
        self.proof_annotations
            .as_ref()
            .is_some_and(|annotations| !annotations.is_empty())
    }

    /// Get location for this node (requires file path context)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn location(&self, file_path: &Path) -> Location {
        debug_assert!(
            file_path.exists(),
            "file_path must exist: {}",
            file_path.display()
        );
        Location {
            file_path: file_path.to_path_buf(),
            span: Span {
                start: BytePos(self.source_range.start),
                end: BytePos(self.source_range.end),
            },
        }
    }
}

// Manual Debug implementation for UnifiedAstNode
impl std::fmt::Debug for UnifiedAstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnifiedAstNode")
            .field("kind", &self.kind)
            .field("lang", &self.lang)
            .field("flags", &self.flags)
            .field("parent", &self.parent)
            .field("first_child", &self.first_child)
            .field("next_sibling", &self.next_sibling)
            .field("source_range", &self.source_range)
            .field("semantic_hash", &self.semantic_hash)
            .field("structural_hash", &self.structural_hash)
            .field("name_vector", &self.name_vector)
            // SAFETY: Accessing the raw field of the metadata union for debug output.
            // This is safe because:
            // 1. We only read the raw bytes for display purposes
            // 2. The metadata union is properly initialized
            // 3. Reading raw bytes does not violate memory safety
            .field("metadata_raw", &unsafe { self.metadata.raw })
            .field("proof_annotations", &self.proof_annotations)
            .finish()
    }
}
